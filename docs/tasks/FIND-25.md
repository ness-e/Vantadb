# Task FIND-25 — create_snapshot sin quiesce/subdirs

## Contrato
test: snapshot durante writes concurrentes → reopen del snapshot es consistente;
recursive copy/link verificado según layout real de data_dir;
`cargo nextest run -p vantadb snapshot` pasa.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/storage/engine/mod.rs:494-596` — create_snapshot ×2 variantes cfg (unix :507 hard_link, windows/wasm :540 copy), list_snapshots, Drop.
- `src/storage/engine/maintenance.rs:36-132` — flush(): insert_lock try_lock_for + drain_hnsw_batch_locked + backend.flush + save_vector_index + checkpoint_seq (patrón ERR-010). Patrón de uso previo a lock: rebuild_vector_index (:509) y compact_layout_bfs (:568) llaman flush() ANTES de tomar insert_lock (lock no reentrante).
- `src/storage/engine/init.rs:159-309` — init_storage: base_path = storage_path; backend Fjall abre en base_path (NO en data_dir); `data_dir = base_path.join("data")` (:298); .vanta.lock y .vanta.schema en base_path.
- `src/lsm.rs:59-160` — SegmentLevel::file_name() = "vstore_L0.vanta"..L3; SegmentRegistry crea solo ARCHIVOS en data_dir.
- `tests/core/snapshot_certification.rs:1194-1319` — patrón de reopen: `VantaEmbedded::open(&snap.path)` donde snap.path = `<base>/snapshots/<name>`; consistencia vía replay completo de vanta.wal (checkpoint_seq vive en el backend, que no se copia → seq=0 → replay total).

**Layout real de data_dir (verificado por grep `data_dir.join(`):**
- Solo ARCHIVOS top-level: `vstore_L{0..3}.vanta`, `vector_index.bin` (+ `.bin.tmp` transitorio), `vanta.wal`.
- ÚNICO subdir: `snapshots/` (creado por el propio create_snapshot — DENTRO de data_dir, debe excluirse de la recursión).
- El backend KV (Fjall/RocksDB) y `.vanta.lock`/`.vanta.schema` viven en `base_path`, hermano de `data/` → NO son parte del snapshot filesystem. Gap separado documentado (deuda, no scope de FIND-25).
- `{storage_path}/wal/archive/` (wal_archiver.rs:54) es dead code (FIND-26) y está bajo base_path, no data_dir.

**Referencias hacia dentro:** create_snapshot llamado desde sdk/builder.rs:253, cli_handlers/snapshot.rs:12, cli_server.rs:3031, vantadb-mcp tools.rs:1514, tests/core/snapshot_certification.rs (×5).

**Veredicto:** cambio local a las 2 variantes cfg de create_snapshot + helper privado de recorrido recursivo. Sin cambios de API pública (misma firma). Riesgo principal: deadlock si se llama flush() con insert_lock ya tomado → evitado siguiendo patrón existente (flush primero, sin lock propio).

## Steps

- ✅ Step 1: Test de regresión RED — snapshot durante writes concurrentes → reopen consistente (tests/core/snapshot_certification.rs). **Nota RED honesta:** el test PASA pre-fix en Windows — el replay total de `vanta.wal` enmascara el tear (`checkpoint_seq` vive en el backend, que no se copia → seq=0 → replay completo). El tear real era conjunto-inconsistente index↔vstore (variante copy) y sin punto-de-consistencia determinista. El test queda como guardia del contrato.
- ✅ Step 2: Fix GREEN — flush() previo (guard read_only) + recorrido recursivo excluyendo `snapshots/` en ambas variantes cfg; docstring con trade-off performance. Helpers: `mirror_data_dir()` + `mirror_file()` (unifica hard_link/copy).
- ✅ Step 3: Verify full y cierre

## Context Save Point

- **Verificación ejecutada:**
  - `cargo nextest run -p vantadb snapshot --test snapshot_certification --ignore-default-filter` → 6/6 PASS (incluye FIND-25 + existentes). El binario es heavy-cert (excluido del default-filter en `.config/nextest.toml`); `--ignore-default-filter` es necesario.
  - `cargo nextest run -p vantadb snapshot` (default filter) → 13 unit tests PASS
  - `cargo check -p vantadb` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` ✅
- **Archivos tocados:** `src/storage/engine/mod.rs`, `tests/core/snapshot_certification.rs`, `docs/Backlog.md` (FIND-25 removida, FIND-33 creada), `docs/avance/activo/core-engine.md`
- **Hallazgo colateral:** FIND-33 — backend KV vive fuera de data_dir; snapshot tras compact_wal() pierde datos. Escalado según stop condition del plan (rediseño >100 líneas).
- **Pendiente para el lead:** verificación mecánica + commit (`feat: FIND-25 — create_snapshot quiesce + recursive mirror`) + ejecutar MCP-34b (prerrequisito satisfecho).
