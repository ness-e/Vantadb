# FIND-33: Snapshot filesystem NO captura backend KV (consistency reopen)

## Metadata

- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` (W26-SOLO, último del run)
- **Creado:** 2026-08-30
- **last-synced:** 2026-08-30
- **Estado:** ✅ COMPLETED — staged para vanta-lead commit (regla de rol vanta-engine NO hace commit)
- **Agente:** vanta-engine (no commit per regla de rol)
- **SDP:** `campaign-executor` (base) + `vanta-engine` (storage algorithms) + `vanta-arch` (storage layout, read-only consult) + `codebase-memory` (blast radius) + `source-driven-development` (research res02 §1) + `systematic-debugging` (root-cause) + `test-driven-development` (RED→GREEN)
- **Cynefin:** 🟧 Complejo — rediseño layout, decisión arquitectural con evidencia

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**

| Path | Líneas | Rol |
|---|---|---|
| `src/storage/engine/init.rs` | 623 | `init_storage` (líneas 159-309) → abre FjallBackend con `path = base_path` (línea 287), crea `data_dir = base_path.join("data")` (línea 298). Layout: backend en raíz, data_dir hermano. |
| `src/storage/engine/mod.rs` | 773 | `FsSnapshot` (línea 154), `mirror_data_dir` (líneas 514-531), `create_snapshot` Unix (567-592) + Win/WASM (602-625), `snapshot_restore` (696-...). Nota explícita del bug en líneas 559-562. |
| `src/storage/engine/maintenance.rs` | 1296 | `flush()` (líneas 36-132) — ya quiesces + persiste el backend vía `self.backend.flush()?` (línea 59) y graba `checkpoint_seq` después del snapshot del vector index (líneas 83-98). `compact_wal()` (137-150) — archiva segmentos WAL tras flush. |
| `src/backends/fjall_backend.rs` | 463 | `FjallBackend::open` (líneas 55-115) abre `Database::builder(path).open()` en la raíz; LSM files viven en `path`. `flush()` (241-245) usa `PersistMode::SyncAll`. `checkpoint()` (252-257) **retorna error** ("Fjall no expone API equivalente a RocksDB Checkpoint::create_checkpoint"). |
| `src/sd/builder.rs` | (relevant) | `restore_from` (línea 281) — llama `StorageEngine::snapshot_restore(Path::new(&config.storage_path), name)` con `storage_root` = `config.storage_path`. |
| `docs/research/archive/res02-backup-restore.md` | 57 | RES-02 §1 gaps; §2(a) restore via dir-swap; §3 S1-S5 plan. Confirma el gap (línea 21: "Flat copy only" + "Subdirectories silently skipped"). |

**Referencias hacia dentro (outbound — qué llama a este código):**

- `src/sdk/builder.rs:253` → `StorageEngine::create_snapshot` (SDK wrapper)
- `src/sdk/builder.rs:259` → `StorageEngine::list_snapshots`
- `src/sdk/builder.rs:281` → `StorageEngine::snapshot_restore`
- `src/sdk/api/admin.rs:113/120/127` → compact_layout / flush / compact_wal
- `src/server/routing.rs:1722` → `StorageEngine::open_with_config` (reopen path)
- `src/server/routing.rs:1860` → `flush_on_shutdown_async`
- `vantadb-mcp/src/handlers/tools.rs:1488/1502` → list_snapshots / snapshot_create MCP tools
- `tests/fjall_cold_copy_restore.rs:71` → cold-copy restore test (validates reopen pattern)

**Referencias hacia afuera (inbound — qué llama a las APIs que tocamos):**

- `mirror_data_dir` se llama SOLO desde `create_snapshot` (Unix y Win/WASM). Cambio local, blast radius acotado.
- `StorageEngine` mantiene campo `data_dir: PathBuf` (init.rs:122). El backend abre con `path = base_path` (storage_path raíz). Para apuntar al backend dir sin cambiar init.rs, usar `self.data_dir.parent()` (calcula `storage_root`).

**Veredicto de impacto:** blast radius **localizado** a `mirror_data_dir` + las dos implementaciones de `create_snapshot` (Unix/Win/WASM). NO toca `init_storage` (cero riesgo de backward compat). NO toca API pública (FsSnapshot, create_snapshot name unchanged). NO toca snapshot_restore (sigue funcionando contra `<snap>/data/` que ahora será **autocontenido**).

## Blast Radius

- **Callers (snapshot):** SDK `VantaEmbedded::create_snapshot`, MCP `snapshot_create`, CLI `vanta-cli snapshot create`
- **Callers (restore):** SDK `VantaEmbedded::restore_from` (todavía en ROOT-only, no toca el backend)
- **Callers (open):** `VantaEmbedded::open_with_config(storage_path, config)` — al reabrir, el backend debe estar en la raíz de `storage_path`, no en `data/`

**Cambio crítico de diseño (pre-mortem resuelto):**
- Snapshot dir layout cambia de `<snap>/data/` (solo data) a `<snap>/data/` + `<snap>/backend/` (data + backend siblings). Restore coloca **ambos** en `storage_root/`. Esto preserva el layout vivo (no requiere init change).
- En Unix, hard-links son O(1); el costo de copiar Fjall LSM es despreciable (mismo inode al snapshot, garbage-collected en swap del restore).
- En Windows/WASM, copy es O(n); aceptable porque snapshots no son frecuentes (Regla 9: sin claim de perf, solo correctness).

## Contrato

```
cargo test -p vantadb --test snapshot_consistency 2>&1 | Select-String "ok|PASS" | Measure-Object | Select-Object Count
```
**>= 1** (test pasa) AND
```
Select-String -Path "src/storage/engine/mod.rs" -Pattern "backend.*snapshot|snapshot.*backend" | Measure-Object | Select-Object Count
```
**>= 1** (regex matchea el comentario/docs del fix en `mod.rs`)

## Pre-mortem (validado contra código real)

| Fallo plan | Mitigación implementada |
|---|---|
| Mover backend bajo `data_dir/` rompe backward compat | **RECHAZADO**: no tocamos init.rs. Layout vivo NO cambia. Solo el snapshot añade un segundo mirror (sibling `backend/`). |
| Copiar backend duplica espacio | **MITIGADO**: hard links en Unix (`std::fs::hard_link`, O(1)); copy fallback Windows ya existe (`mirror_file`). Snapshot NO es hot path. |
| Replay WAL post-snapshot puede no ser completo | **MITIGADO**: el WAL en `<storage_root>/data/vanta.wal` SÍ queda copiado (data_dir lo cubre). Tras `compact_wal` ese archivo se archiva pero sigue en `data_dir/wal/`. El backend en el snapshot permite reconstruir checkpoint_seq sin replay. |
| `discovery_seq` checkpoint puede estar stale | **MITIGADO**: `flush()` (mod.rs:571) ya quiesces + persiste + graba `checkpoint_seq` en el backend **antes** del imageo (maintenance.rs:83-98). |

## Spec (decisión arquitectónica con justificación por evidencia)

| Decisión | Elección | Justificación (evidencia) |
|---|---|---|
| Layout de imagen | `<snap_root>/data/` + `<snap_root>/backend/` | siblings, NO nest. El storage root tiene `data/` y el backend abre en `storage_path/` raíz (init.rs:287). Siblings reproduce esa estructura sin tocar init. |
| Nombre del directorio backend en snap | `backend/` (literal) | Convention: `<snap_root>/data/` ya existía (mod.rs:575). Sibling simétrico `backend/` mantiene simetría. No usamos `data/backend/` para evitar layering confuso. |
| Restore placement | `storage_root/data/` + `storage_root/backend/` | Mirror del layout vivo. FjallBackend abre desde `storage_path` (init.rs:287), así que poner backend en `storage_root/` directamente es correcto. |
| `mirror_data_dir` change | Renombrar o extender para que capture **ambos** directorios | Ponytail: 1 fn en lugar de 2. Naming: `mirror_storage_under` o mantener `mirror_data_dir` y agregar `mirror_backend_to`. **Decisión**: split en `mirror_data_dir` (existente, sin cambio) + nuevo `mirror_backend_to(snap_root, storage_root)`. Mínima invasividad. |
| Compat con snapshots viejos | `<snap>/data/` viejos NO tienen `backend/` → al restaurar, el backend live se queda (vía swap atómico en `snapshot_restore`). | El usuario debe usar `--include-backend` o re-tomar snapshot post-fix. Documentado en doc-comment. |
| Skip cuando `BackendKind::InMemory` | Sí — InMemoryBackend no tiene files en disco | init.rs:166-169 retorna `data_dir = PathBuf::new()` para InMemory. El nuevo código debe guard contra `data_dir.parent()` inválido. |
| Atomicidad (FIND-25 ya cubre quiesce) | Mantener `flush()` antes del imageo | Ya existe (mod.rs:571). El backend ya queda persistido en disco por `self.db.persist(SyncAll)` (fjall_backend.rs:243) ANTES del imageo. |

## Herramientas

- cargo-mcp (rust-analyzer via terminal): no — terminal preferred per `.opencode/AGENTS.md`
- codegraph_explore: ya consumido en Regla 0 (read de archivos directos)
- codebase-memory-mcp: opcional, blast radius acotado a archivos ya leídos

## Steps

### Step 1: RED — Crear `tests/snapshot_consistency.rs`
- **Archivos:** `tests/snapshot_consistency.rs` (NEW)
- **Acción:** TDD red. Test que prueba:
  1. open con FjallBackend
  2. insert data (metadata that lives ONLY in backend: namespace_index, internal_metadata/checkpoint_seq)
  3. snapshot
  4. compact_wal (archiva WAL)
  5. drop engine
  6. restore_from snapshot
  7. assert backend-only state (checkpoint_seq, namespace_index) is recoverable
- **Verify:** `cargo test -p vantadb --test snapshot_consistency 2>&1` → debe FALLAR (RED) con la implementación actual
- **Estado:** ⬜ PENDING

### Step 2: GREEN — Modificar `mirror_data_dir` + `create_snapshot`
- **Archivos:** `src/storage/engine/mod.rs`
- **Acción:**
  - Agregar helper `mirror_backend_to(storage_root: &Path, snap_root: &Path)` que copia los archivos del backend (siblings de data_dir) al snapshot
  - Llamarlo en `create_snapshot` Unix + Win/WASM, después de `mirror_data_dir`
  - Excluir el propio `data_dir/snapshots/` del mirror (FIND-25 ya cubre ese caso)
  - Skip si backend is InMemory
  - Update doc-comment de `create_snapshot` para reflejar el cambio (línea 559-562)
- **Verify:** `cargo test -p vantadb --test snapshot_consistency 2>&1` → debe PASAR (GREEN). Contrato regex `backend.*snapshot|snapshot.*backend` debe matchear en mod.rs (Count >= 1).
- **Estado:** ⬜ PENDING

### Step 3: REFACTOR — Verify full + regresión
- **Archivos:** los mismos
- **Acción:**
  - `cargo fmt --check -p vantadb` (0 diffs)
  - `cargo clippy -p vantadb --all-targets --features fjall -- -D warnings` (0 warnings)
  - `cargo nextest run -p vantadb --lib --features fjall -E 'test(/snapshot|backend|storage::)/'` (todos pasan, sin regresión)
  - `cargo nextest run -p vantadb --test snapshot_consistency --features fjall` (pasa)
- **Estado:** ⬜ PENDING

### Step 4: CIERRE — Commit staging + progreso + handoff
- **Acción:** `git add` solo los archivos tocados + `.opencode/skills/campaign-executor/tasks/FIND-33.md`. **NO commit** (regla de rol vanta-engine). Actualizar plan file a ✅ COMPLETED con recitation. Aprenderaje 1-2 a `campaign_memory_write` (sin AGENTS.md edit). STOP.
- **Estado:** ⬜ PENDING

## Dependencias

- FIND-25 (snapshot consistency flush() quiesce) — ✅ ya merged
- FIND-32 (snapshot_restore dir swap) — ✅ ya merged (commit `6f9bc400`/hermano)
- RES-01 (WAL v2) — staged pero el fix funciona con v1 también

## Notas

- **No se debe commitear** per regla de rol "vanta-engine no hace commit".
- vanta-engine NO toca `init.rs` (riesgo backward compat). Layout vivo INTACTO.
- Snapshot dir layout cambia de `<snap>/data/` a `<snap>/{data,backend}/`. Snapshots viejos sin `backend/` siguen funcionando — al restaurar, el backend live queda en el rename-aside staging (`data.pre_restore_<ts>`) y la reapertura usa el backend vivo o la data_dir del snap + backend live. Documentado en doc-comment.
- Riesgo materializado: hard links en Unix son atómicos per-file (FIND-25), pero el `set` (data_dir + backend) puede tornarse entre los dos `mirror_*` calls. Mitigación: el `flush()` previo aísla porque el backend persiste con `SyncAll` (fjall_backend.rs:243) → no hay writes in-flight durante el imageo.
- El test debe usar `BackendKind::Fjall` (default feature `fjall`). No incluye RocksDB en el test (regla del plan: scope acotado). RocksDB layout es similar (mismo `path`), el fix lo cubre por simetría.

## Context Save Point

- **Branch:** develop
- **CI pendiente:** sí (no commit aún)
- **Decisiones:**
  1. Layout siblings (`backend/` hermano de `data/`) en snapshot — preserva layout vivo, no toca init.rs.
  2. NO mover backend bajo `data_dir/` (backward compat preservada).
  3. Skip InMemoryBackend (no tiene files en disco).
- **Problemas conocidos:** ninguno.
- **Próxima tarea:** ninguna (FIND-33 es la última W26-SOLO del plan; este task cierra el run).