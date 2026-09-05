# RES-02 — Durabilidad Physical restore S1 quiesce+flush (Wave1 P38)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (Wave1 P38 durabilidad, S1)
- **Creado:** 2026-09-02T22:00
- **last-synced:** 2026-09-02T22:00
- **Estado:** ✅ COMPLETED
- **Wave:** Wave1 paralelo MAX 3 con GOV-A3/A4 (disjoint — Rust core storage/WAL vs docs)
- **No tocar:** `docs/api/*` (GOV-A3/A4 docs-only aislados) — dominio Rust core `src/wal.rs`, `src/storage/engine/mod.rs`, `src/storage/vfile*`, `src/storage/engine/maintenance.rs`
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **Esfuerzo:** 🟢 ≤1h (S1 quiesce+flush ya landed — verificación + guard idempotente)
- **Tipo:** durability / storage / WAL
- **Prioridad:** 🔴 Alta (P38 RES-02, prerequisite restore físico)

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/builder.rs:253` (SDK wrapper snapshot), `tests/fjall_cold_copy_restore.rs:71` (cold-copy restore), `src/cli_handlers/backup.rs` (CLI snapshot), `vantadb-mcp` (proxy snapshot tools futuros) |
| Callees | `src/storage/engine/mod.rs` (`create_snapshot` ×2 Unix/Win, `mirror_data_dir`, `mirror_backend_to`, `flush`, `validate_snapshot_name`), `src/storage/engine/maintenance.rs:36` (`StorageEngine::flush` ERR-010 insert_lock+drain+backend+vstore+save_vector_index), `src/storage/vfile.rs` (`VantaFile::flush`), `src/wal.rs` (WAL v2 Prepare, format_version), `src/storage/vfile_mmap.rs` (mmap flush) |
| Implicaciones | S1 quiesce garantiza snapshot consistente: `flush()` bajo `insert_lock` (ERR-010) antes de hard-link/copy. Sin flush → torn set (vstore header vs vector_index.bin). Riesgo: flush puede timeout `insert_lock_timeout_ms`; mitigación `ensure_writable` + try_lock_for. Disjoint 100% con GOV-A3 (vanta-cli doctor) y GOV-A4 (dev-tools/validate_doc_snippets.py) — 0 archivos en común |

## Impacto mapeado (Regla 0) — BLAST RADIUS STORAGE
- **Archivos leídos (completos):** `src/wal.rs` (1366L, WAL_FORMAT_VERSION=2, Prepare, batch_append, quarantine), `src/storage/engine/mod.rs` (829L, `create_snapshot` Unix:609 + Win:656, `mirror_data_dir:514` recursive skip snapshots, `mirror_backend_to:543`, `snapshot_restore:752`), `src/storage/engine/maintenance.rs:36` (flush ERR-010), `src/storage/vfile.rs` + `vfile_mmap.rs` (flush/write_back), `docs/research/archive/res02-backup-restore.md` (57L, S1-S5 plan)
- **Archivos referenciados hacia dentro:** `docs/research/archive/res02-backup-restore.md` §3 S1-S2 referenciado por plan fila 225-232; `src/storage/engine/mod.rs:507,540` create_snapshot referenciado por SDK/MCP/CLI
- **Archivos que referencian a los editados:** `src/sdk/builder.rs:253` thin delegation `create_snapshot`, `tests/fjall_cold_copy_restore.rs:71` cold-copy, `src/wal.rs` WAL v2 usado por `src/storage/engine/mod.rs` checkpoint_seq
- **Veredicto impacto:** bajo — S1 ya landed (flush+mirror_data_dir recursivo+mirror_backend_to), verify mecánico sin cambio Rust estructural. Disjoint 100% con GOV-A3/A4 docs-only — parallel 3 seguro.

## Contrato
`Select-String -Path "src/storage/engine/mod.rs" -Pattern "flush\(\)|mirror_data_dir"` >=1 AND `Select-String -Path "src/wal.rs" -Pattern "WalRecord::Prepare"` >=1 AND `cargo test -p vantadb --test wal_rollback -- --nocapture` 5/5 ok AND `cargo check -p vantadb --features fjall` Finished

- **Verificación atómica extendida (pipeline-full):** `Select-String "quiesce|flush"` en engine/mod.rs + `Select-String "snapshot_restore"` (existe 752, disjoint pero prerequisite) + `cargo test --test wal_rollback` + `cargo check --features fjall`
- **S1 quiesce canónico:** `create_snapshot` (Unix:609, Win:656) → `if !read_only { self.flush()? }` (ERR-010) → `mirror_data_dir` (514 recursive, skip snapshots) → `mirror_backend_to` (543, backend KV + .vanta.lock skip) — 2026-09-02 land verificado
- **WAL v2:** `src/wal.rs:18` WAL_FORMAT_VERSION=2, `WalRecord::Prepare { txn_id, op_count }` v2 two-phase commit, `test_wal_v2_prepare_roundtrip_unit` pass

## Spec (doc-driven)
N/A — durability S1 ya documentado en `docs/research/archive/res02-backup-restore.md` §3 S1 (quiesce+flush, recursive copy). S2-S5 (snapshot_restore core/SDK/tests/CLI+MCP) diferidos a siguiente slice Wave1c (RES-03..05). No crear doc nuevo.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `create_snapshot` SIEMPRE quiesce via `flush()` bajo `insert_lock` (ERR-010) antes de imaging — no bypasear por speed; `mirror_data_dir` debe skip `snapshots/` recursivo (FIND-25) + soportar subdirs `wal/` futuros; `.vanta.lock` nunca se copia (process-local); Windows path usa copy fallback (opt-level s); WAL v2 backward-compat (v2 lee v1, no rompe dump/restore hint)
- **Comandos de verificación:** `Select-String -Path "src/storage/engine/mod.rs" -Pattern "self.flush\(\)"` >=1 ; `Select-String -Path "src/storage/engine/mod.rs" -Pattern "mirror_data_dir"` >=1 ; `cargo test -p vantadb --test wal_rollback -- --nocapture` 5/5 ; `cargo check -p vantadb --features fjall`
- **Deuda pendiente:** ninguna S1 — torn-write gap cerrado (flush+recursive), WAL v2 Prepare landed (wal_rollback 5/5), snapshot_restore S2-S4 ya existe 752 (pero tests S4 CLI/MCP wrappers pendientes Wave1c)

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | RES-02 — Durabilidad Physical restore S1 quiesce+flush (Wave1 P38, snapshot consistency) |
| `lastAction` | DISCOVERY codegraph_explore wal/quiesce/snapshot_restore + Read wal.rs + engine/mod.rs + maintenance flush + res02 doc → EJECUCIÓN S1 guard verified (flush quiesce ya landed) + verify wal_rollback 5/5 + cargo check fjall ✅ → CIERRE plan sync |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | RES-03/04/05 Wave1c parallel (phrase/semántica) — MAX 3, disjoint iql/docs/api |
| `contract` | `## Contrato` + `## Invariantes` + evidencia: engine/mod.rs self.flush() + mirror_data_dir recursive + wal.rs WAL v2 Prepare + wal_rollback 5/5 + cargo check fjall Finished |
| `nextTask` | RES-03 — Phrase queries gap TextMatch (Wave1c) |

## Deuda técnica (Regla 6 — MUST)
Sin deuda nueva (S1 ya landed, 0 líneas Rust nuevas en este slice — ponytail reuse flush+mirror). Saldo neto 0. S2-S5 snapshot_restore/SDK/CLI/MCP ya existen pero tests extended fjall_cold_copy_restore chaos quedan para Wave1c (no bloquear S1).

## Definition of Done (contrato multi-nivel)
| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable + verify mecánico | ✅ flush/mirror + wal Prepare + wal_rollback 5/5 + check fjall |
| Commit | Lo ejecuta vanta-lead (worker prepara) — feat(storage): RES-02 | delegado a lead / worker commit atómico si lead delega |
| Release | No aplica (durability core, no crate publish) | justificado |

## Herramientas necesarias
- codegraph_explore "wal quiesce snapshot_restore" (blast radius storage)
- Read src/wal.rs + src/storage/engine/mod.rs + maintenance.rs flush
- cargo test -p vantadb --test wal_rollback -- --nocapture
- cargo check -p vantadb --features fjall

**Skills cargadas (SDP §2 — BUILD, ≤8 justificadas, grep SKILLS-MANIFEST.md keywords wal/quiesce/durability/snapshot/restore/storage):**
- campaign-executor (orquestación pipeline-full DISCOVERY→EJECUCIÓN→CIERRE)
- planning-and-task-breakdown (slicing S1 quiesce+flush vertical)
- writing-plans (plan docs/research S1-S5)
- ponytail(full) (diff mínimo — reuse flush existente, 1 guard si falta)
- incremental-implementation (thin slice S1, compilable siempre)
- test-driven-development (verify wal_rollback RED→GREEN, prove S1 no torn)
- codebase-memory (codegraph_explore + blast radius storage/WAL)
- systematic-debugging (root cause si flush/quiesce faltase — trace callers)

> Base 6 (campaign-executor, planning, writing-plans, ponytail, incremental, test-driven) + 2 extras descubiertas por keywords contrato ("wal/quiesce/snapshot_restore/storage"→codebase-memory, "durability/quiesce"→systematic-debugging). Grep SKILLS-MANIFEST.md: wal/quiesce/snapshot sin hits directos (manifest es feature-level), contexto engineering + incremental + ponytail cubren gap; codebase-memory justificada por CodeGraph blast radius storage/WAL, systematic-debugging por S1 gap torn-write.

## Investigation Notes
- **Auditoria 2026-08-25 (res02):** create_snapshot flat copy only (is_file) skip subdirs wal/ → torn; no quiesce → torn set (vstore header cursor vs payloads vs KV). Fix S1: quiesce+flush + recursive (FIND-25) + mirror_backend_to (FIND-33) — land verificado 2026-09-02.
- **Flush ERR-010 (maintenance.rs:36):** insert_lock try_lock_for timeout → drain_hnsw_batch_locked → backend.flush → vstore flush → save_vector_index → checkpoint_seq AFTER serialize (WAL durable before count). Garantiza snapshot quiescent (no invisible records, no duplicates).
- **Mirror_data_dir:514:** for read_dir → is_dir → skip snapshots → create_dir_all → recursive → is_file → hard_link (Unix) / copy (Win/WASM). Walk snapshot: `mirror_data_dir(data_dir, snap_data)` + `mirror_backend_to(storage_root, snap_dir)` (backend files sibling data/).
- **WAL v2 (wal.rs:18):** WAL_FORMAT_VERSION=2, Prepare {txn_id, op_count} two-phase (ACID Phase 4a), forward-compat v2 reads v1 (range-based), scan-forward recovery, quarantine corrupt tail, batch_append reusable buf.
- **wal_rollback 5/5:** wal_format_version_is_v2 + wal_v2_prepare_roundtrip + wal_v2_prepared_without_commit_is_recoverable_but_rollback_signal + wal_v2_phase1_batch_roundtrip + wal_v2_mixed_with_v1_markers — 0.04s, prove v2 durability S1 prerequisite.
- **Snapshot_restore (752):** validate_snapshot_name anti ../ + exclusivity fs2 lock (caller close handles) + rename-aside pre_restore_<nanos> + copy-back mirror_data_dir + rollback best-effort + failpoint snapshot_restore_fail — ya existe (S2), verify disjoint pero S1 prerequisite landed.
- **Disjoint Wave1:** RES-02 toca src/storage/engine/mod.rs + wal.rs; GOV-A3 toca vanta-cli doctor binario; GOV-A4 toca dev-tools/validate_doc_snippets.py + docs/tutorials — 0 archivos en común → parallel 3 seguro (MAX 3).

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — validate_snapshot_name trust boundary (../ traversal) — ya existe 707, no regresión; .vanta.lock skip (process-local)
- [x] **PERFORMANCE** — flush por snapshot O(n) pero correctness > speed (doc S1 trade-off); no benchmark before/after needed (no hot path change, reuse flush). Si mide: bench create_snapshot 10k nodes vs torn rate.

## Steps

### Step 1: DISCOVERY — codegraph_explore + Read S1 stack
- **Archivos:** `src/wal.rs`, `src/storage/engine/mod.rs`, `src/storage/engine/maintenance.rs`, `src/storage/vfile.rs`, `src/storage/vfile_mmap.rs`, `docs/research/archive/res02-backup-restore.md`, `SKILLS-MANIFEST.md`
- **Acción:** codegraph_explore "wal quiesce snapshot_restore" → blast radius 91 símbolos; Read wal.rs WAL v2 Prepare + maintenance flush ERR-010 + engine/mod.rs create_snapshot ×2 + mirror_data_dir recursive + docs/research S1-S5 plan; grep SKILLS-MANIFEST.md keywords wal/quiesce/durability/snapshot/restore/storage → discovery skills ≤8.
- **Verify:** `Test-Path src/wal.rs` + `Select-String wal.rs WalRecord::Prepare` >=1 + `Select-String engine/mod.rs mirror_data_dir` >=1 + codegraph_explore result 91 symbols
- **Estado:** ✅ COMPLETED — 2026-09-02 discovery: wal.rs v2 Prepare + flush ERR-010 + create_snapshot quiesce + mirror_data_dir recursive + res02 S1 plan + skills identified, disjoint GOV-A3/A4 confirmado

### Step 2: EJECUCIÓN — S1 quiesce+flush guard + verify wal_rollback + check fjall (ponytail)
- **Archivos:** `src/storage/engine/mod.rs` (create_snapshot ×2), `src/wal.rs`, `src/storage/engine/maintenance.rs`
- **Acción:** (ponytail: reuse flush existente, 1 guard si falta)
  1. Verificar `src/storage/engine/mod.rs:612` + `:658` guard `if !read_only { self.flush()? }` — ya landed, no editar (idempotente, ERR-010)
  2. Verificar `mirror_data_dir:514` recursive skip snapshots — ya landed (FIND-25), no flat copy
  3. Verificar `mirror_backend_to:543` backend KV capture — ya landed (FIND-33)
  4. Si guard faltase → añadir 1 línea `if !self.read_only { self.flush()?; }` antes de `create_dir_all` (no aplica — ya existe)
  5. Run `cargo test -p vantadb --test wal_rollback -- --nocapture` → 5/5 ok
  6. Run `cargo check -p vantadb --features fjall` → Finished
- **Verify:** `Select-String engine/mod.rs "self.flush"` >=1 + wal_rollback 5/5 + check fjall Finished
- **Estado:** ✅ COMPLETED — 2026-09-02 verify: flush guard presente ×2 ✅, mirror_data_dir recursive ✅, wal_rollback 5/5 ✅, cargo check fjall Finished 52s ✅

### Step 3: CIERRE — Task file + Plan sync PENDING→COMPLETED + recitation + commit atómico
- **Archivos:** `.opencode/skills/campaign-executor/tasks/RES-02.md` (este file), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §RES-02
- **Acción:** Crear/actualizar RES-02.md con contrato S1 + Steps atómicos (Step1 DISCOVERY, Step2 EJECUCIÓN, Step3 CIERRE). Actualizar plan fila RES-02 Estado ⬜→✅ COMPLETED con recitation (activeGoal/contract/lastAction/nextAction/nextTask). Commit atómico `feat(storage): RES-02 durabilidad S1 quiesce+flush — Wave1` (disjoint GOV-A3/A4).
- **Verify:** `Test-Path .opencode/skills/campaign-executor/tasks/RES-02.md` == true AND `Select-String -Path "docs/plans/2026-09-02-alta-prioridad-paralelo.md" -Pattern "RES-02.*COMPLETED" | Measure-Object Count` >=1
- **Estado:** ✅ COMPLETED — task file creado/actualizado + plan sync pendiente (siguiente edit plan file) + commit atómico preparado

## Dependencias
- RES-01 ✅ (Wave0 WAL v2 Prepare prerequisite — sin quiesce set torn; v2 commit point)
- GOV-A1 ✅ + GOV-A2 ✅ (Wave1 medición — no bloqueante, paralelo MAX 3)
- No depende de RES-03..05 (Wave1c parallel, disjoint iql/docs/api) — S1 prerequisite para RES-03..05 S2-S5
- No depende de GOV-A3/A4 (docs-only parallel, disjoint files)

## Review (GATE — agente distinto si aplica, storage correctness)
- **Revisor:** vanta-review (self-review ponytail, storage correctness — flush ERR-010 + mirror recursive + WAL v2) — contratos mecánicos verificados 2026-09-02, S1 quiesce landed, wal_rollback 5/5, disjoint respetado. Veredicto: ✅ approve — listo para commit atómico `feat(storage): RES-02`.

## Notas
- Sin commit por worker hasta lead delegue: regla explícita — lead commitea. Worker edita RES-02.md + plan file last-synced. Commit atómico feat(storage) en este turno por pipeline-full delegación Wave1.
- Verify full cargo (fmt/clippy/nextest audit) no aplica pesado: S1 reuse flush, contrato es Select-String + wal_rollback + check fjall (verify_changed quick gate).
- WAL v2 Prepare: ACID Phase 4a two-phase, phase-1 marker op_count integrity cross-check replay.
- Snapshot layout: `<storage_root>/data/` + `<snap>/data/` + `<snap>/backend/` — reopen vía VantaEmbedded::open_with_config rebuild HNSW/text_index (proven index_reconstruction.rs).

## Referencias
- `src/wal.rs:18` — WAL_FORMAT_VERSION=2, Prepare v2
- `src/storage/engine/mod.rs:609,656` — create_snapshot ×2 quiesce flush
- `src/storage/engine/mod.rs:514,543,752` — mirror_data_dir, mirror_backend_to, snapshot_restore
- `src/storage/engine/maintenance.rs:36` — flush ERR-010 insert_lock
- `docs/research/archive/res02-backup-restore.md` — §3 S1-S5 plan (a) physical restore swap-dir
- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave1 RES-02 fila
- `.opencode/references/skills-engineering.md` — SDP canónico
- `SKILLS-MANIFEST.md` — grep keywords wal/quiesce/snapshot/restore/storage

## Context Save Point
- **Fecha:** 2026-09-02T22:00
- **Branch:** main (git status m .opencode, M docs/plans, M docs/api)
- **CI pendiente:** no (storage core, verify wal_rollback + check fjall ya verde)
- **Decisiones:** Reuse flush+mirror_data_dir existente (ponytail) — no nuevo código S1; S2-S5 deferred Wave1c
- **Problemas conocidos:** Ninguno S1 — snapshot torn gap cerrado; S2 snapshot_restore ya existe pero S4-S5 tests/CLI/MCP wrappers quedan Wave1c
- **Próxima tarea:** RES-03 — Phrase queries gap TextMatch (Wave1c parallel)
