# Plan de Ejecución: Backup/Restore Chain (2026-08-25)

> **Inicio:** 2026-08-25
> **Estado:** ⏳ EN PROGRESO
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
- **Estado:** ⬜ PENDING

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
- **Estado:** ⬜ PENDING

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
- **Estado:** ⬜ PENDING

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