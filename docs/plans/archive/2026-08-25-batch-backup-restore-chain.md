# Plan de Ejecución: Backup/Restore Chain (2026-08-25)

> **Campaign ID:** e997e5cf-fc35-4084-9a02-55648a44b7ee
> **Inicio:** 2026-08-25
> **Estado:** ✅ COMPLETADO (3/3)
> **Fuente:** docs/Backlog.md (FIND-25, MCP-34b, FIND-26 — hallazgos de RES-02; confirmación del usuario 2026-08-25)
> **Modo:** FAIL_MODE=stop (cadena secuencial: cada task es prerrequisito de la siguiente)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 3 |
| 🟡 DEFER | 0 (en esta cadena) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬇️ downhill = 3 (research RES-02 ya definió el diseño S1-S5; ejecución directa)

> **Cadena secuencial:** FIND-25 (consistencia create_snapshot) es prerrequisito de MCP-34b (restore tool), y FIND-26 (PITR wiring/removal) se decide después. NO paralelizar.

## Tasks

### Task 1: FIND-25 — create_snapshot sin quiesce/subdirs (data loss 🔴)

- **Appetite:** max 4h
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴 (snapshot torn = restore pierde datos)
- **Archivos clave:** `src/storage/engine/mod.rs:507-566` (create_snapshot ×2 variantes), `src/storage/engine/maintenance.rs` (flush existente)
- **Verificación real:** ✅ CÓDIGO-REAL — Paso 0 + RES-02: (1) copia solo archivos top-level (`read_dir` + `is_file()`, mod.rs:521-527/:554-560) — subdirs NO capturados; (2) sin quiescing ni flush previo — snapshot durante writes puede ser torn (hard-link por archivo es atómico pero el conjunto no). El mecanismo de quiesce correcto YA EXISTE: `flush()` en maintenance.rs:36-132 (insert_lock + drain_hnsw_batch_locked + backend.flush + save_vector_index, patrón ERR-010).
- **Gate Justificación:** data loss potencial en snapshots; fix acotado reutilizando flush() existente.
- **Gate Result:** ✅ DO
- **Contrato:** test: snapshot durante writes concurrentes → reopen del snapshot es consistente (todos los nodos o ninguno); recursive copy/link verificado según layout real de data_dir; `cargo nextest run -p vantadb snapshot` pasa
- **Task file:** `skills/campaign-executor/tasks/FIND-25.md`
- **Estado:** ✅ COMMITTED `fix(storage)` - quiesce via flush + mirror recursivo, snapshot tests 13/13

  **Pre-mortem:**
  - Fallo 1: llamar flush() dentro de create_snapshot causa deadlock si ya hay lock tomado
  - Fallo 2: layout de data_dir tiene subdirs inesperados que rompen el copy
  - Fallo 3: el snapshot dir está DENTRO de data_dir → recursión infinita (snapshots/snapshots/)
    - **Mitigación conocida:** excluir el subdir `snapshots/` de la copia
  - **Stop conditions:** si el layout real de data_dir requiere rediseño del snapshot (más de ~100 líneas), escalar a question.
  - **Cynefin:** 🟨 complicado — storage/durabilidad. **Top 3 riesgos:** (1) deadlock flush; (2) recursión snapshots/; (3) layout desconocido.
  - **Risk Register:**
    | Prob×Impacto | Riesgo | Respuesta | Trigger |
    |--------------|--------|-----------|---------|
    | 🟡×🔴 | deadlock entre flush y snapshot | usar try_lock con timeout como hace flush() | test hang |
    | 🟢×🟡 | recursión snapshots/ | excluir subdir explícitamente | test falla |

### Task 2: MCP-34b — snapshot_restore core + SDK + MCP tool

- **Appetite:** max 1d
- **Esfuerzo:** 🟠
- **Prioridad:** 🟡
- **Archivos clave:** `src/storage/engine/mod.rs` (snapshot_restore nuevo), `src/sdk/builder.rs`, `vantadb-mcp/src/handlers/tools.rs`, `tests/fjall_cold_copy_restore.rs` (patrón), `docs/api/`
- **Verificación real:** ✅ CÓDIGO-REAL — RES-02 verificó: snapshot_restore = 0 ocurrencias; patrón válido probado en tests/fjall_cold_copy_restore.rs:71 (stop→copy→reopen preserva BM25/HNSW/hybrid); diseño completo en docs/research/res02-backup-restore.md §2a/§3 (S2-S5).
- **Gate Justificación:** cierra el gap de restore físico; diseño ya investigado y validado (RES-02); prerrequisito FIND-25 satisfecho al completar Task 1.
- **Gate Result:** ✅ DO
- **Contrato:** `StorageEngine::snapshot_restore(name)` + SDK wrapper + MCP tool `snapshot_restore` (identifier validation + destructive-op confirmation arg); test snapshot→mutate→restore→assert retrieval; failpoint `snapshot_restore_fail`; docs ×2 hash SAME
- **Task file:** `skills/campaign-executor/tasks/MCP-34b.md`
- **Estado:** ✅ COMMITTED `feat(storage)` - restore core+SDK+MCP, snapshot 24/24, mcp 73/73

  **Diseño (de RES-02 §2a):**
  1. Validar name como identifier (anti path-traversal, guard de MCP-34a)
  2. Exclusividad: fail si otro proceso tiene fs2 lock; caller dropea handles primero
  3. Safety: mover data_dir actual → `<snap>/pre_restore_<ts>` (rename atómico same-volume)
  4. Copiar `<snap>/data/*` de vuelta a data_dir fresco
  5. Reabrir engine (rebuild índices desde storage)

  **Pre-mortem:**
  - Fallo 1: restore mientras engine abierto → file locks activos (fs2) bloquean el swap
  - Fallo 2: el snapshot pre-FIND-25 es torn → restore restaura datos inconsistentes
  - Fallo 3: rename cross-volume falla (Windows)
  - **Stop conditions:** si el API de restore exige rediseño del engine handle (Arc<RwLock<Option<Arc<Engine>>>>), documentar y escalar.
  - **Cynefin:** 🟨 complicado — storage destructivo. **Top 3 riesgos:** (1) locks activos; (2) snap torn pre-FIND-25; (3) cross-volume rename.
  - **Risk Register:**
    | Prob×Impacto | Riesgo | Respuesta | Trigger |
    |--------------|--------|-----------|---------|
    | 🔴×🔴 | restore de snap torn | exigir snaps post-FIND-25 o warn explícito | test torn falla |
    | 🟡×🔴 | locks fs2 durante swap | API exige engine cerrado / drop handle | test lock falla |

### Task 3: FIND-26 — PITR dead code: wire o remover

- **Appetite:** max 4h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟢
- **Archivos clave:** `src/wal_archiver.rs:56,:219` (WalArchiver/PitrRestorer), `src/lib.rs:149`, `src/wal_sharded.rs` (rotación de segmentos)
- **Verificación real:** ✅ CÓDIGO-REAL — RES-02: WalArchiver/PitrRestorer son dead code (cero call sites desde engine; solo lib.rs:149 export + tests propios). PITR además necesita base snapshot + replay (modelo base+log) y cobertura de WalRecord para deletes sin verificar.
- **Gate Justificación:** decisión binaria clara (wire vs remove); effort acotado a cualquiera de las dos.
- **Gate Result:** ✅ DO
- **Contrato (wire path):** rotación de segmentos llama archive_segment + test de roundtrip PITR básico. **Contrato (remove path):** wal_archiver.rs eliminado + 0 referencias colgantes + clippy/fmt limpios
- **Task file:** `skills/campaign-executor/tasks/FIND-26.md`
- **Estado:** ✅ COMMITTED `refactor(storage)` - wal_archiver.rs removed, ADR-014 superseded, 432/432

=== RECITATION FIND-26 ===
Campaign ID: e997e5cf-fc35-4084-9a02-55648a44b7ee
Objetivo activo: FIND-26: remover dead code PITR (wal_archiver.rs) con 0 refs colgantes + decisión documentada
Estado: completed
Última acción: Regla 0 completa (496L leídos + grep exhaustivo workspace; solo export lib.rs:149 + Cargo.toml pitr feature + tests propios → STOP CONDITION no disparado). Módulo eliminado, docs vivos actualizados (FEATURES/EXPERIMENTAL_FEATURES/ADR-014 superseded/PRO-FEATURES/UNSAFE_INVENTORY/rules x2/Backlog FIND-26 resuelta + CORE-02 bloqueada-con-nota), verify full PASS
Resultado: OK
Próxima acción: Lead: commit 'refactor: FIND-26 — remove dead-code PITR wal_archiver' (9 archivos + 1 borrado). Cadena del plan TERMINADA (3/3 tasks)
Contrato: 
Próxima tarea si completa: 
=== END RECITATION ===

  **Decisión previa (del lead, basada en RES-02 §2b):** REMOVER. Razones: PITR necesita base+log (prerrequisito grande no planificado), sin consumer identificado hoy, y el dead code confunde (parece feature soportada). Si el owner quiere PITR después, git history lo conserva. Documentar en ADR corto o nota en backlog (Regla 5).
  - **Pre-mortem:** —. **Stop conditions:** si al remover aparecen dependencias ocultas (>5 archivos), escalar a question.
  - **Cynefin:** 🟦 obvio (remove path). **Top 3 riesgos:** (1) refs ocultas; (2) owner quería PITR; (3) tests propios del módulo.

## Waves

- **Secuencial (cadena):** Task 1 (FIND-25) → Task 2 (MCP-34b) → Task 3 (FIND-26)
- MAX_CONCURRENT=1 por dependencia. Sub-agentes NO commitean; el lead verifica mecánico y commitea por tarea.

## Notas

- plan-adjust [2026-08-25]: creado desde docs/Backlog.md. Usuario eligió dirección "Backup/restore chain". Diseño fuente: docs/research/res02-backup-restore.md (RES-02, batch anterior).
- El backlog tiene ~30 filas stale (UX/DAUD/etc. completadas en batches agrupados) — limpieza pendiente fuera de este plan.
- ⬆️ uphill = 0 · ⬇️ downhill = 3

=== RECITATION FIND-25 ===
Campaign ID: e997e5cf-fc35-4084-9a02-55648a44b7ee
Objetivo activo: FIND-25: quiesce (flush ERR-010) + recorrido recursivo en create_snapshot, ambas variantes cfg
Estado: completed
Última acción: Fix implementado en mod.rs (flush previo + mirror_data_dir/mirror_file recursivos, guard read_only, docstrings con trade-off), test de regresión agregado, verify completo PASS, registro en avance + Backlog (FIND-33 colateral), memoria escrita
Resultado: OK
Próxima acción: Lead: commit 'feat: FIND-25 — create_snapshot quiesce + recursive mirror'; luego ejecutar MCP-34b (Task 2 del plan)
Contrato: verificacion: cargo nextest run -p vantadb snapshot --test snapshot_certification --ignore-default-filter -> 6/6 PASS (FIND-25 + existentes); cargo nextest run -p vantadb snapshot -> 13 unit PASS; fmt/clippy -D warnings/check PASS; evidencia: claim 'quiesce cierra tear de conjunto' -> src/storage/engine/mod.rs create_snapshot x2 + maintenance.rs:36-132 (confianza alta); claim 'layout data_dir solo archivos top-level' -> grep data_dir.join( + init.rs:298 (alta); claim 'backend KV fuera de data_dir' -> init.rs:287 FjallBackend::open(path) con path=storage_path raiz (alta); artefactos: task file FIND-25.md, docs/avance/activo/core-engine.md, Backlog FIND-33; invariantes: no tocar wal.rs/vector//backends/; flush() NO duplicado; snapshots/ excluido de la recursion; deuda: test RED no reproduce tear pre-fix en Windows (replay WAL enmascara) - guardia de contrato; FIND-33 pendiente; queda_pendiente: lead verifica mecanico + commit; MCP-34b prerrequisito satisfecho
Próxima tarea si completa: MCP-34b
=== END RECITATION ===

=== RECITATION MCP-34b ===
Campaign ID: e997e5cf-fc35-4084-9a02-55648a44b7ee
Objetivo activo: MCP-34b: snapshot_restore core + SDK wrapper + MCP tool con identifier validation + confirm destructiva
Estado: completed
Última acción: Implementacion completa S1-S6: StorageEngine::snapshot_restore (staging hermano + rollback), VantaEmbedded::restore_from, MCP tool con confirm literal, 4 tests nuevos, docs x2 actualizadas; verify full PASS
Resultado: OK
Próxima acción: Lead: commit; orquestador: ejecutar FIND-26
Contrato: verificacion: cargo nextest run -p vantadb snapshot --ignore-default-filter -> 24/24 PASS; cargo nextest run -p vantadb-mcp --test mcp_tests --ignore-default-filter -> 73/73 PASS; failpoint test (features failpoints) 1/1 PASS; cargo fmt --check OK; cargo clippy --workspace --all-targets --all-features sin warnings | evidencia: claim 'restore recupera estado pre-snapshot y revierte mutaciones' -> tests/core/snapshot_certification.rs test_snapshot_restore_roundtrip PASS (confianza alta); claim 'trust boundary name validado en core y MCP' -> mod.rs validate_snapshot_name + tools.rs guard duplicado, test_snapshot_restore_rejects_unsafe_names PASS (alta); claim 'confirm literal true requerido' -> tools.rs dispatch + mcp_tests test_snapshot_restore_requires_confirmation_and_identifier PASS (alta); claim 'failpoint aborta sin tocar data' -> test_snapshot_restore_failpoint_aborts_before_swap PASS (alta) | artefactos: .opencode/skills/campaign-executor/tasks/MCP-34b.md, src/storage/engine/mod.rs, src/sdk/builder.rs, vantadb-mcp/src/handlers/tools.rs, tests/core/snapshot_certification.rs, vantadb-mcp/tests/mcp_tests.rs, docs/api/EMBEDDED_SDK.md, docs/api/MCP.md | invariantes: no tocar wal.rs/vector//backends/ (respetado); mirror_data_dir intacto; snapshots/ sobrevive el swap (test lo assertiona); CLI fuera de contrato | deuda: en Unix un engine abierto durante restore no falla loud (fork silencioso de estado) - documentado en docstrings EMBEDDED_SDK/MCP como contrato embedded; MCP tool deja el engine del server stale hasta restart - nota explicita en payload de exito | queda_pendiente: lead verifica mecanico + commit 'feat: MCP-34b — snapshot_restore core+SDK+MCP'; luego FIND-26 (Task 3: remover wal_archiver dead code)
Próxima tarea si completa: FIND-26
=== END RECITATION ===
