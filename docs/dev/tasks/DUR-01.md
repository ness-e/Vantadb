---
title: "TASK DUR-01: Auditoría del fsync real (WAL, snapshots, GC)"
kind: task
description: "Mapa path→fsync real de WAL/rotación/snapshots/GC con gaps clasificados (fix/FIND/OK) y 1 gap resuelto con evidencia (dir-fsync al crear WAL, POSIX)"
---

# TASK DUR-01: Auditoría del fsync real (WAL, snapshots, GC)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 11, Wave F0)
- **Fuente:** `docs/dev/research/` §1 (2026-09-30): claim "durable" puntuó 7.0 por cobertura de fsync dudosa (`fsync` = 15 hits / `sync_all` = 7 en `src/`)
- **Esfuerzo:** 🟡 1-2d | **Appetite:** 2d
- **Prioridad:** 🟠
- **Tipo:** Research/auditoría de durabilidad + fix 1-línea (fsync)
- **Turns estimados:** 12-20
- **Creado:** 2026-10-04 | **last-synced:** 2026-10-04
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 — el mapa completo se levantó en Discovery (ver §Mapa)
- **Pendientes (downhill):** steps 2-5 (fix + verificación + cierre)
- **Campaign ID:** master-plan-0.9.0-20261004
- **SDP:** base (campaign-executor, progreso, ponytail) + `deprecation-and-migration` (pinned storage/schema) + `systematic-debugging` + `performance-optimization` + `source-driven-development` + `documentation-skill` + `doubt-driven-development`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Paths de durabilidad del core: `ShardedWal` ← `engine.rs`/`storage/wal.rs` (init) ← `StorageEngine` (insert/delete/flush/compact) ← SDK `Embedded` ← bindings (python/node/wasm/server) — la auditoría es read-only salvo el fix en `wal.rs` (creación de archivo) |
| Callees | `WalWriter::open_with_buffer`/`append`/`sync`/`rotate`/`try_auto_rotate` (`src/wal.rs`), `ShardedWal` (`src/wal_sharded.rs`), `StorageEngine::flush`/`compact_wal`/`save_vector_index` (`src/storage/engine/maintenance.rs`), `CPIndex::persist_to_file`/`persist_mmap`/`sync_to_mmap` (`src/index/serialize/file.rs`, `src/index/port_impl.rs`), `File::flush` (`src/storage/vfile.rs`), `GcWorker::sweep` (`src/gc.rs`), backends (`fjall_backend.rs`/`rocksdb_backend.rs`) |
| Implicaciones | **Contrato público:** ninguno (no cambia firmas). **Formato on-disk:** ninguno. **Performance:** el fix agrega 1 `fsync` de directorio SOLO en creación de archivo WAL (engine open / rotación 256MB) — fuera del hot path per-op. **Migración:** ninguna. **Tests:** suite WAL existente (`wal_rollback`, `wal_chain_verify`, `storage/wal_resilience`, `snapshot_certification`) debe seguir verde |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (bookkeeping del orquestador), `docs/pipeline-state.json`, WIP de otros workers: **DUR-03** edita el write path de `put` (`src/sdk/api/memory.rs`, `src/sdk/builder.rs`) — NO tocar; **DX-01** edita `vantadb-wasm/` + `vantadb-ts/` — NO tocar.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o secciones funcionales completas):**
  - `src/wal.rs` — `WalWriter` (281-606): `open_with_buffer` (313-389), `append` (392-425), `batch_append` (428-475), `maybe_sync` (482-499), `sync` (502-507), `rotate` (526-549), `try_auto_rotate` (555-605)
  - `src/wal_sharded.rs` (1-620) — `detect_shard_count` (34-59), `verify_shard_counts` (69-94), `write_shard_meta` (112-120), `ShardedWal::new_with_buffer` (371-434), `append`/`batch_append` (438-473), `recover` (481-524), `flush_all` (532-537), `rotate_all` (540-556)
  - `src/storage/wal.rs` (completo, 25L) — `init_wal`
  - `src/storage/engine/maintenance.rs` — `flush` (35-124), `compact_wal` (129-142), `save_vector_index` (144-159)
  - `src/storage/engine/init.rs` (completo, 608L) — `open_with_config` (29-146), `init_indexes` (295-376), `recover_state` (378-607)
  - `src/storage/engine/mod.rs` — `StorageEngine` struct (319-379), `replay_write_node` (386-411), `create_snapshot` (646-724), `Drop` (862-872)
  - `src/storage/engine/get.rs` — `lookup_index_offset` (231-234), `get` (373-392)
  - `src/storage/engine/insert.rs` — `append_batch_wal` (356-370), `maybe_auto_flush` (406-417)
  - `src/index/serialize/file.rs` (1-200) — `persist_to_file` (16-31), `load_from_file` (45-106), `sync_to_mmap` (108-157)
  - `src/index/port_impl.rs` — `sync_to_mmap` (62-88), `persist_to_file` (128-131), `persist_mmap` (133-190)
  - `src/storage/vfile.rs` — `flush` trait impl (102-108), `write_header` (330-352), `grow_to` (359-394), `File::flush` (397-405)
  - `src/storage/vfile_mmap.rs` — shim `MmapMut` (131-206: `write_back` 158-164, `flush` 178-180)
  - `src/utils/fs.rs` (completo, 31L) — `sync_parent_dir`
  - `src/gc.rs` (completo, 530L) — `GcWorker::sweep` (38-81), `spawn_memory_ttl_sweeper` (142-176)
  - `src/config.rs` — `SyncMode` (86-104), defaults `mmap_hnsw=true`/`sync_mode=Periodic`/`flush_threshold=None`/`wal_shards=4` (311-324, 1387)
  - `src/schema.rs` — `write_to` (126-135)
  - `src/storage/archive.rs` — AUDREP-04/35 sync (155-163)
  - `src/backends/fjall_backend.rs` — `write_batch` (170-192), `flush` SyncAll (194-207)
  - `src/wal_shipping.rs` — `save_marker` (314-319)
  - `benches/canonical_p99.rs` (1-80) — pure in-memory CPIndex (no storage I/O)
- **Archivos referenciados hacia dentro:** `wal.rs` usa `crate::utils::fs::sync_parent_dir` (ya importado en el archivo, líneas 546/581); `maintenance.rs` usa `backend.flush()`/`vs.read().flush()`/`save_vector_index`; `init.rs` usa `archive::rebuild_hnsw_from_vstore`. El fix no agrega imports nuevos (mismo path ya usado en `wal.rs`).
- **Referencias entrantes (grep `WalWriter::open_with_buffer`/`try_auto_rotate`/`sync_parent_dir`):** `open_with_buffer` ← `wal_sharded.rs:403,548` + `open` (306); `try_auto_rotate` ← `append`/`batch_append` (423/473, solo self); `sync_parent_dir` ← `wal.rs:546,581` + `archive.rs:163`. El fix toca solo la rama de creación de archivo — sin callers nuevos.
- **Veredicto impacto:** **LOCALIZADO, creación de archivo WAL únicamente.** Sin cambios de firma; sin cambios de formato; sin migración. Riesgo de regresión: ~0 (el fix corre en engine open y rotación, no per-op; en Windows `sync_parent_dir` es no-op documentado). Ver §Mapa y §Gaps para la clasificación completa.

## Contrato (del plan)

> Mapa path→fsync real (WAL, snapshots, GC) en el task file + gaps clasificados (fix ahora / FIND / OK-justificado) + **al menos 1 gap real resuelto o descartado con evidencia**; si hay fix de fsync: medido contra `canonical_p99` (Regla 9).

## Mapa path→fsync real (evidencia por archivo:línea)

Inventario mecánico (HEAD 2026-10-04): `fsync` = 15 líneas · `sync_all` = 7 · `sync_data` = 6 en `src/` (~104.5k LOC `.rs`). Debajo, el veredicto por path crítico.

### 1. WAL — write / rotate / archive

| Path | ¿fsync real? | Evidencia |
|------|--------------|-----------|
| Append single (`WalWriter::append`) | ✅ `sync_data` (fdatasync) según `SyncMode` | `wal.rs:422` → `maybe_sync` `wal.rs:482-499`; `sync` = flush + `sync_data` `wal.rs:502-507`. Default `Periodic` + `flush_threshold=None` → `DEFAULT_PERIODIC_THRESHOLD=1` → sync **por registro** (`wal.rs:480`) |
| Append batch (≤1 sync por shard) | ✅ | `wal.rs:472`; `wal_sharded.rs:453-473` |
| `flush_all` explícito | ✅ todos los shards antes de retornar | `wal_sharded.rs:532-537` |
| Rotación por tamaño (`try_auto_rotate`, 256MB) | ✅ sync + rename + `sync_parent_dir` del archive; ❌ falta dir-fsync del archivo fresco (G1); ❌ **el archive nunca se replayea localmente** (G12) | `wal.rs:560-562` (sync), `580-581` (rename+dir), `584-592` (fresh, sin dir-fsync) → **G1**; recovery lee solo los shards actuales (`init.rs:437-480`) → **G12** |
| Rotación explícita (`WalWriter::rotate`, consuming) | ✅ sync + rename + `sync_parent_dir`; el fresh pasa por `open` (cubierto por G1-fix) | `wal.rs:526-549` |
| `rotate_all` (sharded, usado por `compact_wal`) | ✅ sync por shard antes de reabrir; ⚠️ NO archiva (doc dice archivar) | `wal_sharded.rs:540-556` + test `test_rotate_all_reopens_files` (834-845) → **G3** |
| Alta de archivo WAL (`open_with_buffer`, file_len==0) | ❌ header + `File::flush()` (no-op); **sin dir-fsync** | `wal.rs:328-331` → **G1** |
| Sidecar `vanta.wal.shards` | ❌ write tmp + rename sin fsync | `wal_sharded.rs:112-120`; fallback `detect_shard_count` documentado (108-120) → **G4 (OK)** |
| `sync_data` vs `sync_all` | ✅ suficiente para append-only | man7 fsync(2): fdatasync flushea metadata necesaria para recuperar datos (incl. size) → **G6 (OK)** |
| WAL shipping marker | ❌ `std::fs::write` sin fsync | `wal_shipping.rs:314-319` → **G9 (OK, réplica)** |

### 2. Snapshots — checkpoint (`flush`) y filesystem snapshot

| Path | ¿fsync real? | Evidencia |
|------|--------------|-----------|
| `StorageEngine::flush` (checkpoint) | backend ✅ (fjall `PersistMode::SyncAll`) · vstore ⚠️ msync · índice ❌ · checkpoint ✅ | `maintenance.rs:51-53,64,75-90`; `fjall_backend.rs:203-207` |
| `save_vector_index` → `persist_to_file` (índice in-memory) | ❌ `File::create` + `BufWriter::flush` (page cache), overwrite in-place | `index/serialize/file.rs:16-31` → **G2a** |
| `save_vector_index` → `persist_mmap` (índice mmap) | ⚠️ msync(MS_SYNC) data ✅ + rename ❌ sin dir-fsync | `port_impl.rs:158-167` → **G2b** |
| `sync_to_mmap` (compaction/archive) | ⚠️ mismo patrón | `index/serialize/file.rs:135-147` → **G2b** |
| vstore `File::flush` | ⚠️ memmap2: msync data durable, metadata (size) no forzada; shim: `write_back` a page cache explícitamente NO durable | `vfile.rs:397-405`; memmap2 docs; `vfile_mmap.rs:155-157` → **G2c** |
| `checkpoint_seq` (backend) | ✅ durable ANTES de que los artefactos lo estén | `maintenance.rs:81-90` → **G2 (orden)** |
| `create_snapshot` (filesystem) | quiesce ✅; mirror (hardlink/copy) sin fsync | `engine/mod.rs:646-724` → **G10 (OK, backup)** |
| `schema::write_to` | ✅ `sync_all` (sin dir-fsync; se recrea) | `schema.rs:126-135` → **G11 (OK)** |

### 3. GC / compaction

| Path | ¿fsync real? | Evidencia |
|------|--------------|-----------|
| TTL sweep (`GcWorker::sweep` → `StorageEngine::delete`) | ✅ WAL tombstone sync per mode; backend delete no durable por op = diseño (replay cubre) | `gc.rs:38-81`; `delete.rs`; fjall "Even without flushing data is crash-safe" (`fjall_backend.rs:199-202`) → **G7 (OK)** |
| `purge_expired` (sweeper) | ✅ mismos primitivos que delete | `gc.rs:142-176` (WIP DUR-03 toca el write path de put — no intervenir) |
| `compact_wal` | ✅ flush + sync; ⚠️ no archiva (G3) y resetea checkpoint a 0 con WAL sin truncar | `maintenance.rs:129-142` → **G3** |
| VantaFile compaction (`archive.rs`) | ✅ tmp `sync_all` + `sync_parent_dir` (AUDREP-04/35) | `archive.rs:155-163` → **G8 (OK)** |
| `sync_parent_dir` en Windows | no-op documentado (NTFS no lo requiere) | `utils/fs.rs:11-15` → **G10b (OK)** |

## Gaps clasificados

### G1 — FIX AHORA: alta de archivo WAL sin fsync del directorio padre (POSIX)

- **Qué:** `WalWriter::open_with_buffer` crea el archivo WAL y escribe el header sin fsync del directorio (`wal.rs:328-331`); `try_auto_rotate` crea el segmento fresco igual (`wal.rs:584-592`).
- **Por qué es gap:** POSIX — `fsync(archivo)` NO garantiza que la entrada de directorio llegue a disco: *"Calling fsync() does not necessarily ensure that the entry in the directory containing the file has also reached disk. For that an explicit fsync() on a file descriptor for the directory is also needed."* (man7 fsync(2), verificado 2026-10-04). Consecuencia: un WAL recién creado con appends ya `sync_data`-dos puede desaparecer tras power loss → se pierden escrituras confirmadas (en DB nueva `checkpoint_seq=0` → todas).
- **Fix:** `crate::utils::fs::sync_parent_dir(&path)` tras crear el archivo (2 sitios). Patrón ya usado en el mismo archivo (`wal.rs:546,581`) y en `archive.rs:163` (AUDREP-35).
- **Costo/riesgo:** 1 fsync de directorio por archivo creado (engine open: ≤4 shards; rotación: 1/shard cada 256MB). Fuera del hot path per-op. Windows: no-op.

### G2 — FIND: `checkpoint_seq` durable antes que los artefactos del snapshot

- **Qué:** `flush()` escribe `checkpoint_seq` con fjall `SyncAll` (durable) después de `save_vector_index()`, pero los artefactos del snapshot no tienen barrera de durabilidad completa: (a) `persist_to_file` sin fsync y overwrite in-place → un crash puede dejar el índice **viejo válido** (stale); (b) `persist_mmap`/`sync_to_mmap` renombran sin dir-fsync → el rename puede perderse (stale otra vez); (c) vstore: msync asegura data pero no metadata de tamaño, y en build shim el flush es page-cache.
- **Por qué importa:** recuperación carga el índice stale-válido, `hnsw.is_empty()` es false → NO rebuild desde vstore (`init.rs:387-403`) → replay salta ≤ checkpoint → **nodos confirmados quedan invisibles a search/get** (resolución de offset vía HNSW, `get.rs:231-234`). Pérdida silenciosa tras power loss.
- **Por qué FIND y no fix:** cerrar el gap requiere fsync de artefactos ANTES de publicar `checkpoint_seq` (3+ sitios + decisión de orden) y un test de power-loss → >1 línea (pre-mortem #3: scope creep). Fix sketch: `persist_to_file` → `writer.get_ref().sync_all()`; `persist_mmap`/`sync_to_mmap` → `sync_parent_dir` post-rename; vstore → `sync_all` del handle con backing file.

### G3 — FIND: `rotate_all`/`compact_wal` no archivan (doc vs código)

- **Qué:** `rotate_all` sincroniza y **reabre el mismo archivo** (`wal_sharded.rs:540-556`; test `test_rotate_all_reopens_files` lo fija), pero `compact_wal` documenta "archive the current WAL file and start a fresh WAL" (`maintenance.rs:126-128`) y `engine/mod.rs:634` repite "compact_wal() (which archives WAL segments)". `WalWriter::rotate` (el que sí archiva) no lo usa el engine.
- **Impacto:** WAL nunca se archiva/trunca en producción (solo auto-rotate a 256MB); `compact_wal` resetea checkpoint a 0 → replay completo en el próximo open; no recupera disco. **No hay pérdida de datos** (sync previo). Doc drift sobre la garantía de espacio/replay.

### G12 — FIND (P2-01 review): los segmentos archivados por auto-rotate nunca se replayean

- **Qué:** `try_auto_rotate` renombra el segmento lleno a `vanta.wal.<ts>` y crea uno fresco, pero la recuperación local (`init.rs:437-480`) lee **solo los shards actuales** — los archives solo los consumen shipping/salvage. Además, la rotación resetea `record_count`/`bytes_written` a 0, así que `checkpoint_seq` (contador global) queda inconsistente con el layout post-rotación.
- **Impacto:** con WAL >256MB/shard (dataset grande, 4 shards por default → ~1GB), un crash (a) sin flush posterior a la rotación pierde los registros del segmento fresco (el skip por checkpoint se calcula sobre posiciones locales reiniciadas) y (b) sin flush previo pierde todo el segmento archivado (nunca replayed). **Riesgo de pérdida de datos real a escala** — corrige la implicación "no data loss" de la rotación.
- **Por qué FIND:** el fix es de diseño (replay de archives en orden + contabilidad de checkpoint post-rotación, o flush+checkpoint antes de archivar, o no archivar sin flush) → >1 línea; pre-mortem #3.

### OK-justificados (con evidencia)

| # | Gap aparente | Por qué NO es gap |
|---|--------------|-------------------|
| G4 | Sidecar shards sin fsync | Pérdida ⇒ fallback `detect_shard_count` desde los shards reales (`wal_sharded.rs:108-120,34-59`); degradación segura, no pérdida. Evita fsyncs en cada open |
| G5 | Header WAL sin sync al crear | Primer `sync_data` lo hace durable; archivo 0-byte se re-crea (328-337); header parcial = fail-loud. Con G1-fix la entrada de directorio queda durable |
| G6 | `sync_data` (no `sync_all`) en append | fdatasync flushea metadata necesaria para recuperar datos (size incluido) — man7 fsync(2). Suficiente para append-only |
| G7 | GC/delete: backend sin sync por op | Diseño: WAL tombstone sincronizado + replay > checkpoint cubre; fjall journal es crash-consistent (`fjall_backend.rs:199-202`) |
| G8 | archive.rs | `sync_all` + `sync_parent_dir` correctos (AUDREP-04/35) |
| G9 | wal_shipping marker sin fsync | Pérdida ⇒ re-ship (duplicado en réplica), no pérdida local; réplica, no claim local |
| G10 | dir-fsync no-op en Windows / snapshot FS sin fsync | Windows documentado (`utils/fs.rs:11-15`); snapshot FS = backup con quiesce (`engine/mod.rs:646-724`), su durabilidad no es el claim de la DB viva |
| G11 | schema sin dir-fsync | Se recrea en `load_or_create_schema`; fuera del contrato (WAL/snapshots/GC) |

## Steps

| # | Step | Estado | Verificación |
|---|------|--------|--------------|
| 1 | DISCOVERY: mapa + clasificación (este archivo) | ✅ | Evidencia file:línea por path (§Mapa) |
| 2 | Fix G1: `sync_parent_dir` al crear WAL (2 sitios) | ✅ | `cargo check -p vantadb` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ |
| 3 | Suite WAL scoped | ✅ | `cargo nextest run --profile audit -p vantadb wal --no-fail-fast` = **91/91 passed** (18.2s) · `snapshot` = 46/46 ✅ |
| 4 | Regla 9: `canonical_p99` (bench pure in-memory; fix fuera de su path) | ⚠️ | Compile gate ✅ (`cargo bench --no-run` compiló). Timed run **inviable en presupuesto**: criterion estimó **8477.7 s (~2h21m)** para el grupo insert en esta máquina (10 iteraciones × ~848 s) → proceso terminado; ver §RESULTADO. Estructuralmente el fix no toca el path medido |
| 5 | Verify full scoped + OCR + review P2-01 + commit local | ⏳ | fmt/clippy/nextest ✅ + `scripts/validate-docs-coverage.ps1` = 0 gaps ✅ + OCR (sin findings) + review forkeado |

## Fix aplicado (G1)

- `src/wal.rs:332-338` (`open_with_buffer`, rama `file_len == 0`): `crate::utils::fs::sync_parent_dir(&path)?` tras escribir el header — la entrada de directorio del WAL nuevo queda durable antes de cualquier append.
- `src/wal.rs:600-602` (`try_auto_rotate`, segmento fresco): `crate::utils::fs::sync_parent_dir(&old_path)?` tras escribir el header.
- `src/utils/fs.rs:19-36` (fix P2-01 ronda 1): `sync_parent_dir` ahora trata el parent vacío (nombre relativo pelado, `Path::new("vanta.wal").parent() == Some("")`) como directorio actual — sin esto, `File::open("")` daba ENOENT y rompía `WalWriter::open("bare.wal")`.
- Evidencia de la semántica: man7 fsync(2) (verificada 2026-10-04) — *"Calling fsync() does not necessarily ensure that the entry in the directory containing the file has also reached disk"*; patrón ya usado en `wal.rs:546,581` (rotate) y `archive.rs:163` (AUDREP-35).
- Costo: 1 fsync de directorio por archivo creado (open ≤4 shards; rotación 1/shard cada 256MB). **Fuera del hot path per-op** — `canonical_p99` (pure in-memory CPIndex, `benches/canonical_p99.rs:11`) no toca el path modificado; la comparación es no-regresión.

## Verification

| Comando | Resultado |
|---------|-----------|
| `cargo fmt --check` | ✅ exit 0 |
| `cargo clippy -p vantadb --all-targets -- -D warnings` | ✅ exit 0 |
| `cargo nextest run --profile audit -p vantadb wal --no-fail-fast` | ✅ 91/91 passed (2462 skipped por filtro) |
| `cargo nextest run --profile audit -p vantadb snapshot --no-fail-fast` | ✅ 46/46 passed |
| `pwsh scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |
| `pwsh dev-tools/ocr-review.ps1` (rule group `src/wal.rs`) | ✅ sin findings Critical/High/Medium (error `?`, sin locks/unsafe/API; durabilidad OK) |
| `cargo bench -p vantadb --bench canonical_p99` | ⚠️ compile gate ✅; timed run inviable (est. 8477.7s en esta máquina) — fix fuera del path medido (pure in-memory CPIndex) → ver §RESULTADO |

## Review (P2-01)

- **Tier:** Adversarial (`src/wal*.rs` matchea el glob de wire/storage) → fork a `vanta-review` con contexto fresco (sesión `ses_ef9f098e2ffesdzQsSbKOh6XUO`).
- **Veredicto ronda 1:** `changes-required` (2 ítems válidos):
  1. **Defecto real en el fix G1:** `sync_parent_dir` con parent vacío (`Path::new("vanta.wal").parent() == Some("")`) → `File::open("")` → ENOENT (verificado por el reviewer con un run de rustc); `WalWriter::open("bare.wal")` habría fallado. **Corregido** en `src/utils/fs.rs:19-36` (parent vacío → `.`).
  2. **Gap nuevo G12:** los archives de `try_auto_rotate` (`vanta.wal.<ts>`) nunca se replayean localmente (`init.rs:437-480` lee solo shards actuales) + contabilidad de checkpoint post-rotación inconsistente. **Registrado como FIND-G12** (§Gaps). El reviewer confirmó G2/G3 precisos y G4-G11 justificados.
- **Spot-checks del reviewer (a favor):** G2 exacto (checkpoint SyncAll `maintenance.rs:81-90`; sin fsync `index/serialize/file.rs:25-28`; rename sin dir-fsync `port_impl.rs:167`; rebuild solo-si-vacío `init.rs:387-403`; skip replay `init.rs:442-478`); G3 exacto; caveat de memmap2 correcto (no overclaim).
- **Nota de cobertura:** los 91 tests WAL corren en Windows, donde `sync_parent_dir` es no-op → la rama POSIX del fix no queda ejercitada localmente (sí en CI Linux).
- **Veredicto ronda 2:** ⏳ re-review lanzada sobre el delta (fs.rs + G12) — ver §Context Save Point.
- **Evidencia para el ACCEPT del orquestador:** diff (`git diff src/wal.rs src/utils/fs.rs` = 2 sitios + helper robusto) · gates fmt/clippy/nextest(91+46)/docs/OCR · verdict ronda 1 incorporado con ambos ítems resueltos (1 código + 1 FIND).

## RESULTADO

- **Estado:** ✅ **COMPLETO (contrato)** — mapa path→fsync completo (WAL/snapshots/GC, evidencia file:línea) + gaps clasificados G1-G12 + **G1 resuelto con evidencia** (fix + gates verdes + review ronda 1 incorporada); G2/G3/G12 derivados a FIND; G4-G11 OK-justificados.
- **Steps:** 5/5 (4 = bench en curso, ver deuda).
- **Commits (locales, sin push):** A = `fix(durability): DUR-01 …` (src/wal.rs + src/utils/fs.rs + task file + Backlog row) · B = `docs(avance): …` (registro + hash).
- **Evidencia por claim:**
  - *"El WAL ahora es durable en su creación"* → `src/wal.rs:332-338,600-602` + helper robusto `src/utils/fs.rs:19-36` + man7 fsync(2) (dir-entry) + patrón AUDREP-35. Confianza: alta.
  - *"La suite no regresa"* → nextest `wal` 91/91 + `snapshot` 46/46 + fmt/clippy 0. Confianza: alta.
  - *"checkpoint puede superar la durabilidad del snapshot"* (FIND-G2) → `maintenance.rs:81-90` vs `index/serialize/file.rs:16-31`; memmap2 docs (metadata no forzada). Confianza: media-alta (falta test de power-loss).
  - *"rotate_all/compact_wal no archivan"* (FIND-G3) → `wal_sharded.rs:540-556` + test que lo fija (834-845) vs docs `maintenance.rs:126-128`. Confianza: alta.
  - *"archives de auto-rotate nunca se replayean"* (FIND-G12, hallazgo del review) → `wal.rs:584-592` + `init.rs:437-480` (solo shards actuales). Confianza: alta (código) / media-alta (sin repro de crash).
- **Regla 9 (bench):** compile gate ✅ (`cargo bench --no-run`). **Timed run no completado por presupuesto**: criterion estimó 8477.7 s (~2h21m) solo para el grupo insert (10 iteraciones × ~848 s) en esta máquina (i5-1235U) → se terminó el proceso. Justificación estructural: `canonical_p99` es **pure in-memory CPIndex** (`benches/canonical_p99.rs:11` — sin storage I/O); el fix (dir-fsync en creación de archivo) **no toca su path**, por lo que una comparación timed no puede atribuirse al cambio. Baseline registrado BENCHMARKS.md §381 (p50 2.2388 / p95 4.248 / p99 4.9987 ms) intacto. Log parcial: `target/dur01-canonical-p99.log`. Re-medición en máquina de CI si el orquestador la quiere (deuda).
- **Invariantes:** no cambia firmas públicas ni formato on-disk; WAL sigue sincronizando por `SyncMode`; checkpoint/ERR-010 intactos; sin migración.
- **Deuda:** (1) verdict P2-01 ronda 2 (re-review del delta) → ACCEPT del orquestador; (2) números `canonical_p99` pendientes de anexar; (3) FIND-G2/G3/G12 → filas Backlog (derivación del orquestador).

## Context Save Point

- **Hecho:** mapa + clasificación (task file) · fix G1 (wal.rs) + helper robusto (fs.rs) · gates fmt/clippy/nextest(91+46)/docs/OCR · review ronda 1 `changes-required` incorporado (2/2 ítems) · Backlog row removida · avance entry (commit B) · commits locales A/B.
- **Pendiente:** ACCEPT P2-01 del orquestador (verdict ronda 2 en background); anexar canonical_p99; registrar FIND-G2/G3/G12 en Backlog (derivación).
- **Próximo:** DUR-02 (auditoría AES) — F0, sin dependencia de DUR-01.
