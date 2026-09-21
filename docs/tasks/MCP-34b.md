# MCP-34b — snapshot_restore core + SDK + MCP tool

**Estado:** ⏳ IN PROGRESS
**Plan:** docs/plans/2026-08-25-batch-backup-restore-chain.md (Task 2)
**Diseño fuente:** docs/research/res02-backup-restore.md §2a / §3 S2-S5
**Prerrequisito:** FIND-25 ✅ (quiesce via flush + mirror recursivo excluyendo snapshots/)

## Impacto mapeado (Regla 0)

**Archivos leídos completos (secciones relevantes):**
- `src/storage/engine/mod.rs:494-655` — mirror_file/mirror_data_dir (FIND-25), create_snapshot ×2 cfg, list_snapshots, Drop con fs2 unlock. `data_dir` es `pub` (:337).
- `src/sdk/builder.rs:1-263` — VantaEmbedded {engine: Arc<RwLock<Option<Arc<StorageEngine>>>>}, open/open_with_config/close/create_snapshot/list_snapshots.
- `vantadb-mcp/src/handlers/tools.rs:450-530,1440-1522` — schema snapshot_create/list_snapshots; guard MCP-34a: validate_identifier + separadores + '.'/'..'.
- `tests/core/snapshot_certification.rs` completo — patrón harness (TerminalReporter/VantaHarness), seed_snapshot_data, tests FIND-25; snapshots reabren como DB independiente vía VantaEmbedded::open(&snap.path).
- `docs/api/EMBEDDED_SDK.md:259-267` (Snapshots API), `docs/api/MCP.md:171-182` (tool table).
- `vantadb-mcp/tests/mcp_tests.rs:3580-3596,3695-3830` — patrón setup_storage/recovery_call/handle_tools_call, test tools-list.

**Layout verificado (init.rs:298):** `data_dir = <storage_root>/data`; snapshots viven en `<data_dir>/snapshots/<name>/data/`; backend KV fuera de data_dir (WAL replay al reabrir).

**Referencias entrantes:** MCP tool dispatch (tools.rs), cli_handlers/snapshot.rs (solo create/list — NO toco CLI, fuera de contrato).

**Decisión de diseño clave (desviación documentada de RES-02 §2a paso 3):**
RES-02 proponía mover data_dir → `<snap>/pre_restore_<ts>`, pero snapshots/ vive DENTRO de data_dir → el rename anidarían los snapshots dentro de sí mismos. Fix: staging HERMANO de data_dir (`<root>/data.pre_restore_<nanos>`), rename atómico same-volume, luego `staging/snapshots` se mueve de vuelta al data_dir fresco antes del copy-back. Original se conserva hasta que el copy-back completa (rollback en fallo); se elimina solo en éxito.

**Veredicto:** blast radius acotado — 0 símbolos existentes modificados salvo docstring de snapshot_create (MCP-34a "not yet implemented" note). Nuevos símbolos: `StorageEngine::snapshot_restore` (asociada, toma storage_root — exclusividad estructural: sin &self no hay engine abierto desde este handle), `VantaEmbedded::restore_from` (asociada estática), MCP dispatch arm + schema. Sin cambios en wal.rs/vector//storage backends.

## Spec

| Decisión | Elección | Evidencia |
|---|---|---|
| Firma core | `StorageEngine::snapshot_restore(storage_root: &Path, name: &str) -> Result<PathBuf>` — función asociada SIN `&self` | La API embedded exige drop del handle para el swap (fs2 lock); sin instancia es imposible restaurar con locks vivos. RES-02 §2a paso 2 |
| Validación name | guard propio en core (`is_empty`, `.`, `..`, `/`, `\`, control chars) → `InvalidInput`; duplicado en MCP (defense in depth) | Trust boundary; mismo guard que MCP-34a en tools.rs:1507 |
| Snapshot inexistente | `NotFound { kind: "snapshot", id: name }` | error.rs:175 |
| Swap | rename(data_dir → staging hermano) → mkdir fresh data_dir → rename(staging/snapshots → data_dir/snapshots) → mirror_data_dir(snap_data → data_dir) | std::fs::rename atómico same-volume Win+Unix; mirror ya excluye snapshots/ (FIND-25) |
| Fallo post-rename | rollback best-effort: remove partial data_dir + rename(staging → data_dir); error original propagado | op destructiva — nunca dejar DB vacía si hay backup disponible |
| data_dir inexistente | crear fresh + copy-back directo (sin staging) | restore a directorio limpio |
| Failpoint | `snapshot_restore_fail` tras validación/existencia, antes del rename (espejo de snapshot_create_fail mod.rs:580) | contrato |
| SDK | `VantaEmbedded::restore_from(config: VantaConfig, name: &str) -> Result<Self>` asociada: valida+restaura+reopen con open_with_config. Flujo caller: `db.close()?; let db = VantaEmbedded::restore_from(cfg.clone(), "n")?;` | ergonomía más simple correcta; lock fs2 del viejo handle bloquearía el reopen si no se cerró (fail loud en Windows) |
| MCP | params `{name, confirm}`; `confirm != true` literal → error_content pidiendo confirmación (patrón delete confirm mcp_tests:1545); éxito devuelve `{restored, path, note}` avisando reopen necesario | RES-02 S5; server mantiene engine abierto — nota explícita |
| Tests | snapshot_certification.rs: roundtrip (snapshot→mutate→close→restore→assert pre-sí/post-no), traversal ×4, failpoint (cfg failpoints). mcp_tests.rs: confirm requerido, traversal, tools-list | contrato |

## Steps

- ✅ S1: Core `StorageEngine::snapshot_restore` + validate_snapshot_name + failpoint (mod.rs)
- ✅ S2: SDK `VantaEmbedded::restore_from` wrapper (builder.rs)
- ✅ S3: MCP tool schema + dispatch + confirm (tools.rs)
- ✅ S4: Tests core (roundtrip/traversal/failpoint) + MCP tests (confirm/traversal/list)
- ✅ S5: Docs ×2 (EMBEDDED_SDK.md Snapshots API + MCP.md tool table)
- ✅ S6: Verify full + cierre

## Verify (evidencia)

- `cargo nextest run -p vantadb --test snapshot_certification --ignore-default-filter -E 'test(snapshot_restore)'` → 2/2 PASS (roundtrip + unsafe names)
- `cargo nextest run -p vantadb --test snapshot_certification --features failpoints --ignore-default-filter -E 'test(snapshot_restore_failpoint)'` → 1/1 PASS
- `cargo nextest run -p vantadb snapshot --ignore-default-filter` → 24/24 PASS (nuevos + existentes FIND-25/MCP-34a)
- `cargo nextest run -p vantadb-mcp --test mcp_tests --ignore-default-filter` → 73/73 PASS
- `cargo fmt --check` → OK · `cargo clippy --workspace --all-targets --all-features` → sin warnings
- NO COMMIT (el lead verifica mecánico y commitea)

## Context Save Point

Tarea completa. Archivos tocados: mod.rs, builder.rs, tools.rs, snapshot_certification.rs, mcp_tests.rs, docs/api/EMBEDDED_SDK.md, docs/api/MCP.md, task file. Desviación documentada de RES-02 §2a: staging hermano `<root>/data.pre_restore_<nanos>` en vez de `<snap>/pre_restore_<ts>` (snapshots/ vive dentro de data_dir — el rename anidarían los snapshots en sí mismos).
